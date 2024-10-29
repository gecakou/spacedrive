//
//  NativeFunctions.swift
//  Spacedrive
//
//  Created by Arnab Chakraborty on 10/27/24.
//

import Foundation

@objc(NativeFunctions) public class NativeFunctions: NSObject {
    @objc public static func hello(_ resolve: RCTPromiseResolveBlock, rejecter reject: RCTPromiseRejectBlock) {
        let message = "Hello, World!"
        resolve(message)
    }
    
    @objc public func testInput(_ input: String, resolver resolve: RCTPromiseResolveBlock, rejecter reject: RCTPromiseRejectBlock) {
        let message = "Hello, \(input)!"
        resolve(message)
    }
    
    @objc public func getBookmarkUrl(_ input_url: String, resolver resolve: RCTPromiseResolveBlock, rejecter reject: RCTPromiseRejectBlock) {
        // Convert the string into a URL swift can understand
        let url = URL(string: input_url)!
        
        do {
            let bookmarkData = try url.bookmarkData(options: .minimalBookmark, includingResourceValuesForKeys: nil, relativeTo: nil)
            
            let bookmarkUrl = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent("bookmark.url")
            try bookmarkData.write(to: bookmarkUrl)
            
            resolve(bookmarkUrl.absoluteString)
        } catch {
            reject("Error", error.localizedDescription, error)
        }
    }
    
    @objc public func getReadableBookmarkUrl(_ input_url: String, resolver resolve: RCTPromiseResolveBlock, rejecter reject: RCTPromiseRejectBlock) {
        let url = URL(string: input_url)!
        
        do {
            let bookmarkData = try Data(contentsOf: url)
            
            var isStale = false
            let url = try URL(resolvingBookmarkData: bookmarkData, bookmarkDataIsStale: &isStale)
            
            guard !isStale else {
                let error = NSError(domain: "com.reactnative.bookmark", code: 1, userInfo: nil)
                
                let errorMessage = "The bookmark is stale. Please try again."
                reject("Error", errorMessage, error)
                return
            }
            
            let readableUrl = url.absoluteString
            resolve(readableUrl)
    
        } catch {
            reject("Error", error.localizedDescription, error)
        }
    }
 
    @objc static func requiresMainQueueSetup() -> Bool {
        return true
    }
}
