// Deobfuscated AnyList API patterns from original.min.js
// This file contains cleaned-up code samples showing how the web client works

// ============================================================================
// AUTHENTICATION & TOKEN MANAGEMENT
// ============================================================================

class AuthTokenManager {
    constructor() {
        this.accessToken = null;
        this.refreshToken = null;
        this.isRefreshing = false;
        this.queuedRequests = [];
    }

    async refreshAccessToken() {
        if (this.isRefreshing) {
            // Wait for current refresh to complete
            return new Promise((resolve, reject) => {
                this.queuedRequests.push({ resolve, reject });
            });
        }

        this.isRefreshing = true;

        try {
            const response = await fetch('/auth/token/refresh', {
                method: 'POST',
                body: JSON.stringify({
                    refresh_token: this.refreshToken
                })
            });

            const data = await response.json();
            this.accessToken = data.access_token;
            this.refreshToken = data.refresh_token;

            // Process queued requests
            this.queuedRequests.forEach(req => req.resolve(this.accessToken));
            this.queuedRequests = [];

            return this.accessToken;
        } catch (error) {
            this.queuedRequests.forEach(req => req.reject(error));
            this.queuedRequests = [];
            // Notify: ALAuthTokenManagerDidDetectInvalidRefreshToken
            throw error;
        } finally {
            this.isRefreshing = false;
        }
    }

    getHeaders() {
        return {
            'Authorization': `Bearer ${this.accessToken}`,
            'X-AnyLeaf-API-Version': '3',
            'X-AnyLeaf-Client-Identifier': this.getClientId(),
            'Content-Type': 'application/x-protobuf'
        };
    }

    getClientId() {
        // Generate once per client, store in localStorage/config
        if (!this.clientId) {
            this.clientId = uuid4();
        }
        return this.clientId;
    }
}

// ============================================================================
// WEBSOCKET CONNECTION MANAGER
// ============================================================================

class WebSocketManager {
    constructor(accessToken) {
        this.accessToken = accessToken;
        this.retryDelay = 500; // Start at 500ms
        this.maxRetryDelay = 120000; // Max 120 seconds
        this.ws = null;
        this.heartbeatInterval = null;
        this.missedHeartbeats = 0;
        this.maxMissedHeartbeats = 3;
    }

    connect() {
        const encodedToken = encodeURIComponent(this.accessToken);
        const wsUrl = `wss://www.anylist.com/data/add-user-listener?access_token=${encodedToken}`;

        this.ws = new WebSocket(wsUrl);

        this.ws.onopen = () => {
            console.log('WebSocket connected');
            this.retryDelay = 500; // Reset retry delay on successful connection
            this.startHeartbeat();
            // Notify: ALWebSocketConnectionStateDidChangeNotification
        };

        this.ws.onmessage = (event) => {
            if (event.data === '--heartbeat--') {
                this.handleHeartbeat();
            } else if (event.data === 'refresh-shopping-lists') {
                this.handleRefreshShoppingLists();
            } else {
                console.log('WebSocket message:', event.data);
            }
        };

        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };

        this.ws.onclose = (event) => {
            console.log('WebSocket closed:', event.code, event.reason);
            this.stopHeartbeat();
            this.scheduleReconnect();
        };
    }

    handleHeartbeat() {
        this.missedHeartbeats = 0;
        // Send heartbeat response
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send('--heartbeat--');
        }
    }

    startHeartbeat() {
        // Check for heartbeats from server
        this.heartbeatInterval = setInterval(() => {
            this.missedHeartbeats++;
            if (this.missedHeartbeats >= this.maxMissedHeartbeats) {
                console.error('Too many missed heartbeats, force closing WebSocket');
                this.ws.close();
            }
        }, 30000); // Check every 30 seconds
    }

    stopHeartbeat() {
        if (this.heartbeatInterval) {
            clearInterval(this.heartbeatInterval);
            this.heartbeatInterval = null;
        }
    }

    handleRefreshShoppingLists() {
        // Notify: ALUserDidRefreshShoppingListNotification
        console.log('Server requested shopping list refresh');
        // Trigger data refetch
    }

    scheduleReconnect() {
        console.log(`Retrying WebSocket connection in ${this.retryDelay / 1000} seconds`);
        setTimeout(() => {
            this.connect();
        }, this.retryDelay);

        // Exponential backoff
        this.retryDelay = Math.min(this.retryDelay * 2, this.maxRetryDelay);
    }

    close() {
        if (this.ws) {
            this.ws.close();
        }
        this.stopHeartbeat();
    }
}

// ============================================================================
// OPERATION QUEUE MANAGER
// ============================================================================

class OperationQueue {
    constructor(endpoint, operationType, operationListType) {
        this.endpoint = endpoint;
        this.operationType = operationType;
        this.operationListType = operationListType;
        this.pendingOperations = [];
        this.maxConcurrent = 5;
        this.isSyncing = false;
    }

    addOperation(operation) {
        // Add metadata to operation
        operation.metadata = {
            operationId: uuid4(),
            handlerId: operation.handlerId,
            timestamp: Date.now() / 1000,
            logicalTimestamp: this.getNextLogicalTimestamp(),
            operationClass: operation.operationClass || 0
        };

        this.pendingOperations.push(operation);

        // Trigger sync if not already syncing
        if (!this.isSyncing) {
            this.sync();
        }
    }

    async sync() {
        if (this.isSyncing || this.pendingOperations.length === 0) {
            return;
        }

        this.isSyncing = true;

        try {
            // Create operation list
            const operationList = {
                operations: this.pendingOperations
            };

            // Encode as protobuf
            const encoded = this.operationListType.encode(operationList).finish();

            // Send to server
            const response = await fetch(`https://www.anylist.com${this.endpoint}`, {
                method: 'POST',
                headers: authManager.getHeaders(),
                body: encoded
            });

            const responseBuffer = await response.arrayBuffer();
            const editResponse = PBEditOperationResponse.decode(new Uint8Array(responseBuffer));

            // Process response - match operation IDs and update timestamps
            const processedIds = editResponse.processedOperations || [];
            processedIds.forEach(timestamp => {
                const opIndex = this.pendingOperations.findIndex(
                    op => op.metadata.operationId === timestamp.identifier
                );
                if (opIndex !== -1) {
                    // Operation was processed successfully
                    this.pendingOperations.splice(opIndex, 1);
                }
            });

            // Notify: ALEditOperationsQueueDidChangeSyncStateNotification
        } catch (error) {
            console.error('Sync error:', error);
            // Keep operations in queue for retry
        } finally {
            this.isSyncing = false;

            // If there are still pending operations, retry
            if (this.pendingOperations.length > 0) {
                setTimeout(() => this.sync(), 1000);
            }
        }
    }

    getNextLogicalTimestamp() {
        // Increment logical timestamp for conflict resolution
        return Date.now();
    }
}

// ============================================================================
// SHOPPING LIST OPERATIONS
// ============================================================================

class ShoppingListManager {
    constructor() {
        this.updateQueue = new OperationQueue(
            '/data/shopping-lists/update-v2',
            'PBListOperation',
            'PBListOperationList'
        );
    }

    createNewList(name) {
        const operation = {
            handlerId: 'new-shopping-list',
            listId: uuid4(),
            list: {
                identifier: uuid4(),
                timestamp: Date.now() / 1000,
                name: name,
                items: [],
                creator: currentUserId,
                sharedUsers: [],
                logicalClockTime: 1,
                allowsMultipleListCategoryGroups: true,
                listItemSortOrder: 0, // Manual
                newListItemPosition: 0  // Bottom
            }
        };

        this.updateQueue.addOperation(operation);
    }

    addItemToList(listId, itemName, quantity = null, details = null) {
        const operation = {
            handlerId: 'add-shopping-list-item',
            listId: listId,
            listItemId: uuid4(),
            listItem: {
                identifier: uuid4(),
                serverModTime: Date.now() / 1000,
                listId: listId,
                name: itemName,
                quantity: quantity,
                details: details,
                checked: false,
                userId: currentUserId,
                manualSortIndex: 0
            }
        };

        this.updateQueue.addOperation(operation);
    }

    checkOffItem(listId, itemId) {
        const operation = {
            handlerId: 'set-list-item-checked',
            listId: listId,
            listItemId: itemId,
            updatedValue: 'true',
            originalValue: 'false'
        };

        this.updateQueue.addOperation(operation);
    }

    deleteItem(listId, itemId) {
        const operation = {
            handlerId: 'remove-shopping-list-item',
            listId: listId,
            listItemId: itemId
        };

        this.updateQueue.addOperation(operation);
    }

    shareList(listId, email) {
        // Uses special endpoint: /data/shopping-lists/share-list
        const operation = {
            handlerId: 'share-shopping-list',
            listId: listId,
            // sharedUser email added here
        };

        // Send directly to share endpoint instead of update queue
        return this.sendShareOperation(operation);
    }

    async sendShareOperation(operation) {
        const encoded = PBListOperation.encode(operation).finish();
        const response = await fetch('https://www.anylist.com/data/shopping-lists/share-list', {
            method: 'POST',
            headers: authManager.getHeaders(),
            body: encoded
        });

        const responseBuffer = await response.arrayBuffer();
        return PBShareListOperationResponse.decode(new Uint8Array(responseBuffer));
    }
}

// ============================================================================
// RECIPE OPERATIONS
// ============================================================================

class RecipeManager {
    constructor() {
        this.updateQueue = new OperationQueue(
            '/data/user-recipe-data/update',
            'PBRecipeOperation',
            'PBRecipeOperationList'
        );
    }

    createRecipe(name, ingredients, preparationSteps) {
        const operation = {
            handlerId: 'new-recipe', // Inferred
            recipe: {
                identifier: uuid4(),
                timestamp: Date.now() / 1000,
                name: name,
                ingredients: ingredients.map(ing => ({
                    name: ing.name,
                    quantity: ing.quantity,
                    note: ing.note
                })),
                preparationSteps: preparationSteps,
                scaleFactor: 1.0,
                creationTimestamp: Date.now() / 1000
            }
        };

        this.updateQueue.addOperation(operation);
    }

    deleteRecipe(recipeId) {
        const operation = {
            handlerId: 'remove-recipe',
            recipeIds: [recipeId]
        };

        this.updateQueue.addOperation(operation);
    }

    createRecipeCollection(name) {
        const operation = {
            handlerId: 'new-recipe-collection',
            recipeCollection: {
                identifier: uuid4(),
                timestamp: Date.now() / 1000,
                name: name,
                recipeIds: []
            }
        };

        this.updateQueue.addOperation(operation);
    }

    addRecipeToCollection(collectionId, recipeId) {
        const operation = {
            handlerId: 'add-recipes-to-collection',
            // Collection and recipe IDs
        };

        this.updateQueue.addOperation(operation);
    }
}

// ============================================================================
// MEAL PLANNING OPERATIONS
// ============================================================================

class MealPlanningManager {
    constructor() {
        this.updateQueue = new OperationQueue(
            '/data/meal-planning-calendar/update',
            'PBCalendarOperation',
            'PBCalendarOperationList'
        );
    }

    createEvent(calendarId, date, recipeId = null, title = null, labelId = null) {
        const operation = {
            handlerId: 'new-event',
            calendarId: calendarId,
            updatedEvent: {
                identifier: uuid4(),
                logicalTimestamp: 1,
                calendarId: calendarId,
                date: date,
                title: title,
                recipeId: recipeId,
                labelId: labelId,
                recipeScaleFactor: 1.0
            }
        };

        this.updateQueue.addOperation(operation);
    }

    deleteEvent(calendarId, eventId) {
        const operation = {
            handlerId: 'delete-event',
            calendarId: calendarId,
            eventIds: [eventId]
        };

        this.updateQueue.addOperation(operation);
    }
}

// ============================================================================
// STORE & CATEGORY OPERATIONS
// ============================================================================

class CategoryManager {
    constructor() {
        this.updateQueue = new OperationQueue(
            '/data/shopping-lists/update-v2',
            'PBListOperation',
            'PBListOperationList'
        );
    }

    createStore(listId, storeName) {
        const operation = {
            handlerId: 'new-store',
            listId: listId,
            updatedStore: {
                identifier: uuid4(),
                logicalTimestamp: 1,
                listId: listId,
                name: storeName,
                sortIndex: 0
            },
            operationClass: 1 // StoreOperation
        };

        this.updateQueue.addOperation(operation);
    }

    deleteStore(listId, storeId) {
        const operation = {
            handlerId: 'delete-store',
            listId: listId,
            originalValue: storeId,
            operationClass: 1 // StoreOperation
        };

        this.updateQueue.addOperation(operation);
    }

    createCategory(listId, categoryGroupId, categoryName) {
        const operation = {
            handlerId: 'create-category',
            listId: listId,
            updatedCategory: {
                identifier: uuid4(),
                logicalTimestamp: 1,
                categoryGroupId: categoryGroupId,
                listId: listId,
                name: categoryName,
                sortIndex: 0
            },
            operationClass: 3 // ListCategoryOperation
        };

        this.updateQueue.addOperation(operation);
    }
}

// ============================================================================
// RATE LIMITER
// ============================================================================

class RateLimiter {
    constructor(name, maxConcurrent) {
        this.name = name;
        this.maxConcurrent = maxConcurrent;
        this.active = 0;
        this.queue = [];
    }

    async execute(fn) {
        if (this.active >= this.maxConcurrent) {
            // Queue the request
            await new Promise(resolve => this.queue.push(resolve));
        }

        this.active++;
        try {
            return await fn();
        } finally {
            this.active--;
            // Process next queued request
            if (this.queue.length > 0) {
                const next = this.queue.shift();
                next();
            }
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function uuid4() {
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
        const r = Math.random() * 16 | 0;
        const v = c === 'x' ? r : (r & 0x3 | 0x8);
        return v.toString(16);
    });
}

// ============================================================================
// EXAMPLE USAGE
// ============================================================================

/*
// Initialize
const authManager = new AuthTokenManager();
const wsManager = new WebSocketManager(authManager.accessToken);
const listManager = new ShoppingListManager();
const recipeManager = new RecipeManager();
const mealPlanManager = new MealPlanningManager();

// Create a list
listManager.createNewList('Weekly Groceries');

// Add items
listManager.addItemToList(listId, 'Milk', '1 gallon');
listManager.addItemToList(listId, 'Eggs', '12');

// Create a recipe
recipeManager.createRecipe('Pasta', [
    { name: 'Pasta', quantity: '1 lb' },
    { name: 'Tomato Sauce', quantity: '2 cups' }
], [
    'Boil water',
    'Cook pasta',
    'Add sauce'
]);

// Add to meal plan
mealPlanManager.createEvent(calendarId, '2025-10-25', recipeId, null, 'dinner-label');

// Connect WebSocket for real-time updates
wsManager.connect();
*/
