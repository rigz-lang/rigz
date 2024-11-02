import XCTest
import SwiftTreeSitter
import TreeSitterRigz

final class TreeSitterRigzTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_rigz())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading Rigz grammar")
    }
}
