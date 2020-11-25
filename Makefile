test: xmllint
	cargo test

xmllint:
	@$(foreach file, $(wildcard xsd/tests/xsd/*.xsd), xmllint --noout --schema $(file) $(file:.xsd=.xml);)
