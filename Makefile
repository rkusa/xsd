test: xmllint
	cargo test

xmllint:
	@$(foreach file, $(wildcard xsd/tests/xsd/*.xsd), xmllint --noout --schema $(file) $(file:.xsd=.xml);)

# Usage: make create_test name=...
create_test:
	touch xsd/tests/xsd/$(name).xml
	touch xsd/tests/xsd/$(name).xsd
	touch xsd/tests/$(name).rs
