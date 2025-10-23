Feature: Chargebacks
  As a trading platform
  I want to handle chargebacks for disputed transactions
  So that fraudulent transactions can be reversed

  Scenario: Chargeback a disputed transaction
    Given a new database
    And I process a deposit of 100.0 for client 1 with transaction id 1
    And I dispute transaction 1 for client 1
    When I chargeback transaction 1 for client 1
    Then the available balance for client 1 should be 0.0
    And the held balance for client 1 should be 0.0
    And the total balance for client 1 should be 0.0
    And the account for client 1 should be locked

  Scenario: Cannot chargeback a non-disputed transaction
    Given a new database
    And I process a deposit of 100.0 for client 1 with transaction id 1
    When I attempt to chargeback transaction 1 for client 1
    Then the transaction should fail with "Transaction is not disputed"

  Scenario: Cannot process transactions on a locked account
    Given a new database
    And I process a deposit of 100.0 for client 1 with transaction id 1
    And I dispute transaction 1 for client 1
    And I chargeback transaction 1 for client 1
    When I attempt to process a deposit of 50.0 for client 1 with transaction id 2
    Then the transaction should fail with "Account is locked"

  Scenario: Cannot chargeback an already charged back transaction
    Given a new database
    And I process a deposit of 100.0 for client 1 with transaction id 1
    And I dispute transaction 1 for client 1
    And I chargeback transaction 1 for client 1
    When I attempt to chargeback transaction 1 for client 1
    Then the transaction should fail with "Transaction already charged back"

  Scenario: Can dispute transactions on locked accounts
    Given a new database
    And I process a deposit of 100.0 for client 1 with transaction id 1
    And I process a deposit of 200.0 for client 1 with transaction id 2
    And I dispute transaction 1 for client 1
    And I chargeback transaction 1 for client 1
    When I dispute transaction 2 for client 1
    Then the available balance for client 1 should be 0.0
    And the held balance for client 1 should be 200.0
    And the account for client 1 should be locked