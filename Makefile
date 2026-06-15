.PHONY: build release test check clean install uninstall restart run fmt fmt-fix clippy doc help dist install-service uninstall-service

# 默认目标
all: build

# 构建（debug 模式）
build:
	cargo build

# 构建（release 模式）
release:
	cargo build --release

# 运行测试
test:
	cargo test

# 代码检查（编译 + 测试 + clippy）
check: fmt clippy test

# 格式化代码
fmt:
	cargo fmt -- --check

# Clippy 静态分析
clippy:
	cargo clippy -- -D warnings

# 安装到用户目录（默认 ~/.local/bin，无需 sudo）
PREFIX ?= $(HOME)/.local
install: release
	install -d $(PREFIX)/bin
	install -m 755 target/release/keyflow $(PREFIX)/bin/keyflow

# 卸载
uninstall:
	rm -f $(PREFIX)/bin/keyflow

# 安装 systemd 用户服务并启用
install-service: install
	install -d $(HOME)/.config/systemd/user
	install -m 644 packaging/systemd/keyflow.service $(HOME)/.config/systemd/user/keyflow.service
	systemctl --user daemon-reload
	systemctl --user enable keyflow.service
	@echo "Service installed. Start with: systemctl --user start keyflow"

# 卸载 systemd 用户服务
uninstall-service:
	-systemctl --user stop keyflow.service 2>/dev/null
	-systemctl --user disable keyflow.service 2>/dev/null
	rm -f $(HOME)/.config/systemd/user/keyflow.service
	systemctl --user daemon-reload
	@echo "Service uninstalled."

# 打包发布 tarball（含二进制、service、config、安装 Makefile、README）
DIST_NAME := keyflow-$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')-x86_64-linux
dist: release
	@mkdir -p dist/$(DIST_NAME)
	@cp target/release/keyflow dist/$(DIST_NAME)/
	@cp packaging/systemd/keyflow.service dist/$(DIST_NAME)/
	@cp keyflow.toml.example dist/$(DIST_NAME)/
	@cp packaging/dist/Makefile dist/$(DIST_NAME)/
	@cp packaging/dist/README.md dist/$(DIST_NAME)/
	@cp README.md dist/$(DIST_NAME)/
	@cp LICENSE dist/$(DIST_NAME)/ 2>/dev/null || true
	@tar -czf dist/$(DIST_NAME).tar.gz -C dist $(DIST_NAME)
	@rm -rf dist/$(DIST_NAME)
	@echo "Package created: dist/$(DIST_NAME).tar.gz"

# 停止 → 编译安装 → 重启 daemon
restart:
	@-keyflow stop 2>/dev/null || true
	@sleep 0.5
	$(MAKE) install
	@echo "Starting daemon..."
	@keyflow run

# 清理构建产物
clean:
	cargo clean

# 运行（开发模式）
run:
	cargo run

# 格式化代码（不检查）
fmt-fix:
	cargo fmt

# 生成文档
doc:
	cargo doc --open

# 显示帮助
help:
	@echo "KeyFlow — 非粘贴型密码框辅助输入工具"
	@echo ""
	@echo "可用命令:"
	@echo "  make build             构建项目（debug 模式）"
	@echo "  make release           构建项目（release 模式）"
	@echo "  make test              运行所有测试"
	@echo "  make check             代码检查（格式 + clippy + 测试）"
	@echo "  make fmt               检查代码格式"
	@echo "  make fmt-fix           自动格式化代码"
	@echo "  make clippy            运行 clippy 静态分析"
	@echo "  make install           安装到 ~/.local/bin（无需 sudo）"
	@echo "  make uninstall         从 ~/.local/bin 卸载"
	@echo "  make install-service   安装 systemd 用户服务并启用"
	@echo "  make uninstall-service 卸载 systemd 用户服务"
	@echo "  make restart           停止旧 daemon → 编译安装 → 启动新 daemon"
	@echo "  make dist              打包发布 tarball"
	@echo "  make clean             清理构建产物"
	@echo "  make run               开发模式运行"
	@echo "  make doc               生成并打开文档"
	@echo "  make help              显示此帮助信息"
	@echo ""
	@echo "自定义安装路径:"
	@echo "  make install PREFIX=/opt/keyflow"
