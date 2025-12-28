cd ~/repos/univrs-dol/target/bootstrap/src

# Show the end of each problematic function
echo "=== lex (line 69) - end of function ==="
sed -n '69,85p' lexer.rs | tail -5

echo "=== advance (line 108) - end of function ==="
sed -n '108,125p' lexer.rs | tail -5

echo "=== scan_identifier (line 182) - end of function ==="
sed -n '182,205p' lexer.rs | tail -5

echo "=== scan_number (line 210) - end of function ==="
sed -n '210,265p' lexer.rs | tail -5

echo "=== scan_string (line 301) - end of function ==="
sed -n '301,340p' lexer.rs | tail -5

echo "=== scan_token (line 342) - end of function ==="
sed -n '342,400p' lexer.rs | tail -5
