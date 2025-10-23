Feature: Basic Transaction Processing
  As a trading platform
  I want to process basic deposits and withdrawals
  So that clients can manage their account balances

  Scenario: Process a deposit transaction
    Given a new database
    When I process a deposit of 100.0 for client 1 with transaction id 1
    Then the available balance for client 1 should be 100.0
    And the held balance for client 1 should be 0.0
    And the total balance for client 1 should be 100.0
    And the account for client 1 should not be locked

  Scenario: Process a withdrawal with sufficient funds
    Given a new database
    And I process a deposit of 100.0 for client 1 with transaction id 1
    When I process a withdrawal of 50.0 for client 1 with transaction id 2
    Then the available balance for client 1 should be 50.0
    And the held balance for client 1 should be 0.0
    And the total balance for client 1 should be 50.0

  Scenario: Attempt withdrawal with insufficient funds
    Given a new database
    And I process a deposit of 50.0 for client 1 with transaction id 1
    When I attempt to process a withdrawal of 100.0 for client 1 with transaction id 2
    Then the transaction should fail with "Insufficient funds"
    And the available balance for client 1 should be 50.0

  Scenario: Multiple deposits accumulate correctly
    Given a new database
    When I process a deposit of 25.0 for client 1 with transaction id 1
    And I process a deposit of 75.0 for client 1 with transaction id 2
    Then the available balance for client 1 should be 100.0
    And the total balance for client 1 should be 100.0