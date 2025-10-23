Feature: Negative Balance Handling
  Background:
    Negative balances can occur when disputes happen after withdrawals,
    representing scenarios where customers have spent more than is currently available.

  Scenario: Dispute deposit after withdrawal causes negative available balance
    Given a new database
    When I process a deposit of 100.0 for client 1 with transaction id 1
    And I process a withdrawal of 50.0 for client 1 with transaction id 2
    And I dispute transaction 1 for client 1
    Then the available balance for client 1 should be -50.0
    And the held balance for client 1 should be 100.0
    And the total balance for client 1 should be 50.0
    And the account for client 1 should not be locked

  Scenario: Cannot withdraw from negative available balance
    Given a new database
    When I process a deposit of 100.0 for client 1 with transaction id 1
    And I process a withdrawal of 75.0 for client 1 with transaction id 2
    And I dispute transaction 1 for client 1
    Then the available balance for client 1 should be -75.0
    And the held balance for client 1 should be 100.0
    When I attempt to process a withdrawal of 50.0 for client 1 with transaction id 3
    Then the transaction should fail with "Insufficient funds"
    And the available balance for client 1 should be -75.0

  Scenario: Resolve disputed transaction restores positive balance
    Given a new database
    When I process a deposit of 100.0 for client 1 with transaction id 1
    And I process a withdrawal of 30.0 for client 1 with transaction id 2
    And I dispute transaction 1 for client 1
    Then the available balance for client 1 should be -30.0
    And the held balance for client 1 should be 100.0
    When I resolve transaction 1 for client 1
    Then the available balance for client 1 should be 70.0
    And the held balance for client 1 should be 0.0
    And the total balance for client 1 should be 70.0

  Scenario: Chargeback from negative available balance
    Given a new database
    When I process a deposit of 100.0 for client 1 with transaction id 1
    And I process a withdrawal of 80.0 for client 1 with transaction id 2
    And I dispute transaction 1 for client 1
    Then the available balance for client 1 should be -80.0
    And the held balance for client 1 should be 100.0
    When I chargeback transaction 1 for client 1
    Then the available balance for client 1 should be -80.0
    And the held balance for client 1 should be 0.0
    And the total balance for client 1 should be -80.0
    And the account for client 1 should be locked

  Scenario: Multiple deposits and withdrawals with dispute
    Given a new database
    When I process a deposit of 50.0 for client 1 with transaction id 1
    And I process a deposit of 75.0 for client 1 with transaction id 2
    And I process a withdrawal of 60.0 for client 1 with transaction id 3
    And I process a withdrawal of 40.0 for client 1 with transaction id 4
    Then the available balance for client 1 should be 25.0
    When I dispute transaction 2 for client 1
    Then the available balance for client 1 should be -50.0
    And the held balance for client 1 should be 75.0
    And the total balance for client 1 should be 25.0