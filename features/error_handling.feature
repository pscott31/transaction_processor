Feature: Invalid String Parsing Error Handling
  As a trading platform
  I want to handle invalid string formats gracefully
  So that the system remains robust against bad input

  Scenario: Handle empty string
    Given a new database
    When I attempt to process a deposit of "" for client 1 with transaction id 1
    Then the transaction should fail with "Invalid amount format"

  Scenario: Handle invalid characters
    Given a new database  
    When I attempt to process a deposit of "abc" for client 1 with transaction id 1
    Then the transaction should fail with "Invalid amount format"

  Scenario: Handle too many decimal places
    Given a new database
    When I attempt to process a deposit of "123.12345" for client 1 with transaction id 1
    Then the transaction should fail with "Invalid amount format"

  Scenario: Handle multiple decimal points
    Given a new database
    When I attempt to process a deposit of "12.34.56" for client 1 with transaction id 1
    Then the transaction should fail with "Invalid amount format"