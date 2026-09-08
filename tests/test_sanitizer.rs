use rtb::modules::xlsx::clean_sql_query;

#[test]
fn test_powershell_backtick_continuation() {
    let ps = "SELECT o.ID, c.Name `\r\n FROM Orders o `\r\n JOIN Customers c ON o.CustID = c.ID `\r\n WHERE o.Total > 500;";
    let cleaned = clean_sql_query(ps);
    assert_eq!(cleaned, "SELECT o.ID, c.Name FROM Orders o JOIN Customers c ON o.CustID = c.ID WHERE o.Total > 500");
}

#[test]
fn test_cmd_caret_continuation() {
    let cmd = "SELECT * ^\r\n FROM Employees ^\r\n WHERE Department = 'Engineering';";
    let cleaned = clean_sql_query(cmd);
    assert_eq!(cleaned, "SELECT * FROM Employees WHERE Department = 'Engineering'");
}

#[test]
fn test_bash_backslash_continuation() {
    let bash = "SELECT u.Username, o.Amount \\\n FROM Users u \\\n JOIN Orders o ON u.ID = o.UserID \\\n WHERE o.Status = 'DELIVERED';";
    let cleaned = clean_sql_query(bash);
    assert_eq!(cleaned, "SELECT u.Username, o.Amount FROM Users u JOIN Orders o ON u.ID = o.UserID WHERE o.Status = 'DELIVERED'");
}

#[test]
fn test_smart_and_curly_quotes_normalization() {
    let query = "SELECT * FROM “Orders” WHERE Region = ‘North America’;";
    let cleaned = clean_sql_query(query);
    assert_eq!(cleaned, "SELECT * FROM \"Orders\" WHERE Region = 'North America'");
}

#[test]
fn test_preserves_special_characters_inside_quotes() {
    let query = "SELECT * FROM Users WHERE Email = 'bob\\special`caret^char@domain.com'";
    let cleaned = clean_sql_query(query);
    assert_eq!(cleaned, "SELECT * FROM Users WHERE Email = 'bob\\special`caret^char@domain.com'");
}
