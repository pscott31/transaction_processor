Feature: Transaction Disputes
  As a trading platform
  I want to handle transaction disputes
  So that clients can resolve payment issues

  Scenario: Dispute a deposit transaction
    Given a new database
    And I process a deposit of 100.0 for client 1 with transaction id 1
    When I dispute transaction 1 for client 1
    Then the available balance for client 1 should be 0.0
    And the held balance for client 1 should be 100.0
    And the total balance for client 1 should be 100.0

  Scenario: Cannot dispute a withdrawal transaction
    Given a new database
    And I process a deposit of 100.0 for client 1 with transaction id 1
    And I process a withdrawal of 50.0 for client 1 with transaction id 2
    When I attempt to dispute transaction 2 for client 1
    Then the transaction should fail with "Withdrawal transaction cannot be disputed"

  Scenario: Cannot dispute an already disputed transaction
    Given a new database
    And I process a deposit of 100.0 for client 1 with transaction id 1
    And I dispute transaction 1 for client 1
    When I attempt to dispute transaction 1 for client 1
    Then the transaction should fail with "Transaction already disputed"

  Scenario: Resolve a disputed transaction
    Given a new database
    And I process a deposit of 100.0 for client 1 with transaction id 1
    And I dispute transaction 1 for client 1
    When I resolve transaction 1 for client 1
    Then the available balance for client 1 should be 100.0
    And the held balance for client 1 should be 0.0
    And the total balance for client 1 should be 100.0

  Scenario: Cannot resolve a non-disputed transaction
    Given a new database
    And I process a deposit of 100.0 for client 1 with transaction id 1
    When I attempt to resolve transaction 1 for client 1
    Then the transaction should fail with "Transaction is not disputed"