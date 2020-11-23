test: xmllint
	cargo test --verbose

xmllint: tests/xsd/*.xsd
	$(foreach file, $(wildcard tests/xsd/*.xsd), @xmllint --noout --schema $(file) $(file:.xsd=.xml);)