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

// Writing replaces the item instead of updating it: a new item belongs to the app that creates it,
// so FrameLy can read it later without macOS asking for the keychain password.
func keychainSet(_ account: String, _ secret: String) throws {
    let removed = SecItemDelete(baseQuery(account) as CFDictionary)
    guard removed == errSecSuccess || removed == errSecItemNotFound else { throw keychainError(removed) }
    var add = baseQuery(account)
    add[kSecValueData as String] = Data(secret.utf8)
    add[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
    add[kSecAttrLabel as String] = "FrameLy — \(account)"
    let status = SecItemAdd(add as CFDictionary, nil)
    guard status == errSecSuccess else { throw keychainError(status) }
}

/// Whether an item exists, reading only its attributes (never the secret), so it never prompts.
func keychainExists(_ account: String) throws -> Bool {
    var query = baseQuery(account)
    query[kSecReturnAttributes as String] = true
    query[kSecMatchLimit as String] = kSecMatchLimitOne
    var item: CFTypeRef?
    let status = SecItemCopyMatching(query as CFDictionary, &item)
    if status == errSecItemNotFound { return false }
    guard status == errSecSuccess else { throw keychainError(status) }
    return true
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

@_cdecl("fl_keychain_exists")
public func fl_keychain_exists(_ account: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar> {
    let a = String(cString: account)
    return respond { try keychainExists(a) }
}

@_cdecl("fl_keychain_delete")
public func fl_keychain_delete(_ account: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar> {
    let a = String(cString: account)
    return respond { () throws -> Bool in
        try keychainDelete(a)
        return true
    }
}
