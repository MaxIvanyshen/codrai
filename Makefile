.PHONY: install
install:
	@echo "Installing codr..."
	@chmod +x ./install.sh
	@./install.sh

.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@echo "Clean complete."

.PHONY: uninstall
uninstall:
	@echo "Uninstalling codr..."
	@sudo rm -f /usr/local/bin/codr
	@sudo rm -rf /usr/local/bin/codrai
	@echo "Uninstallation complete. You may need to remove the PATH entry from your .zshrc manually."

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make          - Same as 'make install'"
	@echo "  make install  - Install codr"
	@echo "  make clean    - Remove build artifacts"
	@echo "  make uninstall - Remove codr from your system"

