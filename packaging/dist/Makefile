# KeyFlow 安装 Makefile
# 用于 tarball 发布包的安装、卸载、升级

.PHONY: install uninstall upgrade help

PREFIX ?= $(HOME)/.local
CONFIG_DIR := $(HOME)/.config/keyflow

# 默认目标
all: install

# 安装
install: install-binary install-config install-service
	@echo ""
	@echo "✓ KeyFlow 安装完成"
	@echo ""
	@echo "下一步:"
	@echo "  1. 编辑配置: $(CONFIG_DIR)/keyflow.toml"
	@echo "  2. 启动服务: systemctl --user start keyflow"
	@echo "  3. 查看状态: systemctl --user status keyflow"

# 安装二进制
install-binary:
	install -d $(PREFIX)/bin
	install -m 755 keyflow $(PREFIX)/bin/keyflow

# 安装配置文件（不覆盖已有配置）
install-config:
	@if [ -f $(CONFIG_DIR)/keyflow.toml ]; then \
		echo "配置文件已存在，跳过: $(CONFIG_DIR)/keyflow.toml"; \
	else \
		install -d $(CONFIG_DIR); \
		install -m 644 keyflow.toml.example $(CONFIG_DIR)/keyflow.toml; \
		echo "配置文件已安装: $(CONFIG_DIR)/keyflow.toml"; \
	fi

# 安装 systemd 用户服务
install-service:
	install -d $(HOME)/.config/systemd/user
	install -m 644 keyflow.service $(HOME)/.config/systemd/user/keyflow.service
	systemctl --user daemon-reload
	@echo "systemd 服务已安装"

# 卸载
uninstall: stop-service uninstall-service uninstall-binary
	@echo ""
	@echo "✓ KeyFlow 已卸载"
	@echo "  配置文件保留于: $(CONFIG_DIR)/"

# 停止服务
stop-service:
	@-systemctl --user stop keyflow.service 2>/dev/null || true

# 卸载 systemd 服务
uninstall-service:
	-systemctl --user disable keyflow.service 2>/dev/null
	rm -f $(HOME)/.config/systemd/user/keyflow.service
	systemctl --user daemon-reload

# 卸载二进制
uninstall-binary:
	rm -f $(PREFIX)/bin/keyflow

# 升级（停止服务 → 安装 → 启动服务）
upgrade: stop-service install-binary install-service
	@-systemctl --user start keyflow.service 2>/dev/null || true
	@echo ""
	@echo "✓ KeyFlow 已升级并重启"

# 显示帮助
help:
	@echo "KeyFlow 安装 Makefile"
	@echo ""
	@echo "用法:"
	@echo "  make install    安装（二进制 + 配置 + 服务）"
	@echo "  make uninstall  卸载（保留配置文件）"
	@echo "  make upgrade    升级（停止 → 安装 → 启动）"
	@echo ""
	@echo "自定义安装路径:"
	@echo "  make install PREFIX=/opt/keyflow"
