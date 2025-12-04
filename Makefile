NAME ?= jico
VERSION ?= 0.0.1
RPMBUILD := $(CURDIR)/target/rpm

.PHONY: rpm
rpm:
	cargo build --release
	mkdir -p $(RPMBUILD)/SOURCES
	tar -czf $(RPMBUILD)/SOURCES/$(NAME)-$(VERSION).tar.gz \
		--exclude './target' \
		--transform 's,^,$(NAME)-$(VERSION)/,' \
		.
	rpmbuild -bb packaging/$(NAME).spec \
		--define "_topdir $(RPMBUILD)" \
		--define "version $(VERSION)"
