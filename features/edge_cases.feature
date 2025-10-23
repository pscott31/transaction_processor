Feature: Edge Cases
  As a trading platform
  I want to handle edge cases properly
  So that the system is robust and reliable

  Scenario: Process transactions for multiple clients
    Given a new database
    When I process a deposit of 100.0 for client 1 with transaction id 1
    And I process a deposit of 200.0 for client 2 with transaction id 2
    Then the available balance for client 1 should be 100.0
    And the available balance for client 2 should be 200.0

  Scenario: Cannot dispute non-existent transaction
    Given a new database
    When I attempt to dispute transaction 999 for client 1
    Then the transaction should fail with "Transaction not found"

  Scenario: Handle small decimal amounts correctly
    Given a new database
    When I process a deposit of 0.0001 for client 1 with transaction id 1
    Then the available balance for client 1 should be 0.0001

  Scenario: Handle zero amount transactions
    Given a new database
    When I attempt to process a deposit of 0.0 for client 1 with transaction id 1
    Then the transaction should fail with "Amount must be positive"

  Scenario: Transaction with same ID for different clients
    Given a new database
    When I process a deposit of 100.0 for client 1 with transaction id 1
    And I process a deposit of 200.0 for client 2 with transaction id 1
    Then the available balance for client 1 should be 100.0
    And the available balance for client 2 should be 200.0