Feature: Advanced Transaction Scenarios
  As a trading platform
  I want to test complex transaction combinations
  So that the system handles all edge cases correctly

  Scenario: Multiple transactions and their interactions
    Given a new database
    When I process a deposit of 100.0 for client 1 with transaction id 1
    And I process a deposit of 200.0 for client 1 with transaction id 2
    And I process a withdrawal of 50.0 for client 1 with transaction id 3
    Then the available balance for client 1 should be 250.0
    And the total balance for client 1 should be 250.0

  Scenario: Dispute and resolve multiple transactions
    Given a new database
    And I process a deposit of 100.0 for client 1 with transaction id 1
    And I process a deposit of 200.0 for client 1 with transaction id 2
    When I dispute transaction 1 for client 1
    Then the available balance for client 1 should be 200.0
    And the held balance for client 1 should be 100.0
    When I dispute transaction 2 for client 1
    Then the available balance for client 1 should be 0.0
    And the held balance for client 1 should be 300.0
    When I resolve transaction 1 for client 1
    Then the available balance for client 1 should be 100.0
    And the held balance for client 1 should be 200.0

  Scenario: Partial chargebacks with multiple transactions
    Given a new database
    And I process a deposit of 100.0 for client 1 with transaction id 1
    And I process a deposit of 200.0 for client 1 with transaction id 2
    And I dispute transaction 1 for client 1
    When I chargeback transaction 1 for client 1
    Then the available balance for client 1 should be 200.0
    And the held balance for client 1 should be 0.0
    And the total balance for client 1 should be 200.0
    And the account for client 1 should be locked