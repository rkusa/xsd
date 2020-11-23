test: xmllint
	cargo test --verbose

xmllint:
	$(foreach file, $(wildcard xsd/tests/xsd/*.xsd), @xmllint --noout --schema $(file) $(file:.xsd=.xml);)