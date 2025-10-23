Feature: String Parsing for Fixed4 Precision
  As a trading platform
  I want to parse string amounts with precise decimal handling
  So that I can avoid floating-point precision issues

  Scenario: Parse whole numbers
    Given a new database
    When I process a deposit of 100 for client 1 with transaction id 1
    Then the available balance for client 1 should be 100.0

  Scenario: Parse numbers with 1 decimal place
    Given a new database
    When I process a deposit of 12.5 for client 1 with transaction id 1
    Then the available balance for client 1 should be 12.5

  Scenario: Parse numbers with 2 decimal places
    Given a new database
    When I process a deposit of 99.99 for client 1 with transaction id 1
    Then the available balance for client 1 should be 99.99

  Scenario: Parse numbers with 3 decimal places
    Given a new database
    When I process a deposit of 123.456 for client 1 with transaction id 1
    Then the available balance for client 1 should be 123.456

  Scenario: Parse numbers with 4 decimal places (maximum precision)
    Given a new database
    When I process a deposit of 0.1234 for client 1 with transaction id 1
    Then the available balance for client 1 should be 0.1234

  Scenario: Parse very small amounts
    Given a new database
    When I process a deposit of 0.0001 for client 1 with transaction id 1
    Then the available balance for client 1 should be 0.0001

  Scenario: Parse amounts starting with decimal point
    Given a new database
    When I process a deposit of .5 for client 1 with transaction id 1
    Then the available balance for client 1 should be 0.5

  Scenario: Large whole numbers
    Given a new database
    When I process a deposit of 1000000 for client 1 with transaction id 1
    Then the available balance for client 1 should be 1000000.0

  Scenario: Combine different precision amounts
    Given a new database
    When I process a deposit of 100 for client 1 with transaction id 1
    And I process a deposit of 0.01 for client 1 with transaction id 2
    And I process a deposit of 12.3456 for client 1 with transaction id 3
    Then the available balance for client 1 should be 112.3556