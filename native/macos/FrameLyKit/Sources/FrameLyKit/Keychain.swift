import Foundation
import Security

// API keys of optional external AI providers live only in the login Keychain (this device,
// available after first unlock). They are never written to settings, databases or logs.

private let service = "app.framely.desktop.ai"

private func baseQuery(_ account: String) -> [String: Any] {
    [kSecClass as String: kSecClassGenericPassword, kSecAttrService as String: service, kSecAttrAccount as String: account]
}

private func keychainError(_ status: OSStatus) -> FFIError {
    let message = (SecCopyErrorMessageString(status, nil) as String?) ?? "OSStatus \(status)"
    return FFIError.platform("Keychain: \(message)")
}

func keychainSet(_ account: String, _ secret: String) throws {
    let data = Data(secret.utf8)
    let update: [String: Any] = [kSecValueData as String: data]
    var status = SecItemUpdate(baseQuery(account) as CFDictionary, update as CFDictionary)
    if status == errSecItemNotFound {
        var add = baseQuery(account)
        add[kSecValueData as String] = data
        add[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
        add[kSecAttrLabel as String] = "FrameLy — \(account)"
        status = SecItemAdd(add as CFDictionary, nil)
    }
    guard status == errSecSuccess else { throw keychainError(status) }
}

func keychainGet(_ account: String) throws -> String? {
    var query = baseQuery(account)
    query[kSecReturnData as String] = true
    query[kSecMatchLimit as String] = kSecMatchLimitOne
    var item: CFTypeRef?
    let status = SecItemCopyMatching(query as CFDictionary, &item)
    if status == errSecItemNotFound { return nil }
    guard status == errSecSuccess, let data = item as? Data else { throw keychainError(status) }
    return String(decoding: data, as: UTF8.self)
}

func keychainDelete(_ account: String) throws {
    let status = SecItemDelete(baseQuery(account) as CFDictionary)
    guard status == errSecSuccess || status == errSecItemNotFound else { throw keychainError(status) }
}

@_cdecl("fl_keychain_set")
public func fl_keychain_set(_ account: UnsafePointer<CChar>, _ secret: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar> {
    let (a, s) = (String(cString: account), String(cString: secret))
    return respond { () throws -> Bool in
        try keychainSet(a, s)
        return true
    }
}

@_cdecl("fl_keychain_get")
public func fl_keychain_get(_ account: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar> {
    let a = String(cString: account)
    return respond { try keychainGet(a) }
}

@_cdecl("fl_keychain_delete")
public func fl_keychain_delete(_ account: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar> {
    let a = String(cString: account)
    return respond { () throws -> Bool in
        try keychainDelete(a)
        return true
    }
}
