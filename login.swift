// http post \
// https://www.anylist.com/data/validate-login \
// X-AnyLeaf-API-Version:3 X-AnyLeaf-Client-Identifier:dskjghf \
// email=anylist@phil.denhoff.ca password='i!e3bY9GKcDA*xXNG_49MPkU ]	\
// --form

import Foundation

struct LoginResponse: Decodable {
  let is_premium_user: Bool
  let signed_user_id: String
  let user_id: String
}

func main() {
  var result: Result<LoginResponse, Error>!
  let semaphore = DispatchSemaphore(value: 0)

  let url = URL(string: "https://www.anylist.com/data/validate-login")!
  var request = URLRequest(url: url)

  request.setValue(
    "3",
    forHTTPHeaderField: "X-AnyLeaf-API-Version"
  )
  request.setValue(
    "asejfklasdfj",
    forHTTPHeaderField: "X-AnyLeaf-Client-Identifier"
  )
  request.setValue(
    "application/x-www-form-urlencoded",
    forHTTPHeaderField: "Content-Type"
  )

  let email = CommandLine.arguments[1]
  let password = CommandLine.arguments[2]

  var requestBodyComponents = URLComponents()
  requestBodyComponents.queryItems = [
    URLQueryItem(name: "email", value: email),
    URLQueryItem(name: "password", value: password),
  ]

  request.httpMethod = "POST"
  request.httpBody = requestBodyComponents.query?.data(using: .utf8)

  // Create the HTTP request
  let session = URLSession.shared
  let task = session.dataTask(with: request) { (data, response, error) in
    if let error = error {
      print("error")
      print(error)
      result = .failure(error)
    } else if let data = data {
      let decoder = try! JSONDecoder().decode(LoginResponse.self, from: data)
      result = .success(decoder)
    } else {
      print("Unexpected error")
      result = .failure(
        NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "Unexpected error"]))
    }

    semaphore.signal()
  }

  task.resume()
  semaphore.wait()

  switch result {
  case .success(let responseBody):
    print("Is premium user?: \(responseBody.is_premium_user)")
    print("Signed user ID: \(responseBody.signed_user_id)")
  case .failure(let error):
    print("Error: \(error)")
  case .none:
    print("ah shit")
  }
}

main()
