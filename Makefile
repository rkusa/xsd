test: xmllint
	cargo test

XML_FILES := $(wildcard xsd/tests/xsd/*.xml)
xmllint: ${XML_FILES}

xsd/tests/xsd/%.xml: xsd/tests/xsd/%.xsd
	xmllint --noout --schema $< $@

# Usage: make create_test name=...
create_test:
	touch xsd/tests/xsd/$(name).xml
	touch xsd/tests/xsd/$(name).xsd
	touch xsd/tests/$(name).rs
