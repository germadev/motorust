# Atajos para desarrollar motorust. `make` a secas muestra la lista.

.DEFAULT_GOAL := help
.PHONY: help install up preview build test bench linux-deps

CARGO ?= cargo
# Como root (contenedores, CI) no hace falta sudo, y a veces ni existe.
SUDO := $(shell [ "$$(id -u)" = 0 ] || echo sudo)

# Lo que Bevy necesita para compilar en Linux: compilador de C++, pkg-config,
# audio (ALSA), mandos (udev), X11 y Wayland.
APT_PACKAGES := g++ pkg-config libx11-dev libasound2-dev libudev-dev \
	libxkbcommon-x11-0 libwayland-dev libxkbcommon-dev
DNF_PACKAGES := gcc-c++ pkgconf-pkg-config libX11-devel alsa-lib-devel \
	systemd-devel wayland-devel libxkbcommon-devel
PACMAN_PACKAGES := pkgconf libx11 libxcursor libxrandr libxi alsa-lib \
	wayland libxkbcommon

help: ## Muestra esta ayuda
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) \
		| awk -F ':.*## ' '{ printf "  make %-8s %s\n", $$1, $$2 }'

install: ## Instala las dependencias del sistema (en Linux) y descarga las crates
	@command -v $(CARGO) >/dev/null \
		|| { echo "Falta Rust: instálalo con rustup (https://rustup.rs)"; exit 1; }
	@if [ "$$(uname -s)" = Linux ]; then $(MAKE) --no-print-directory linux-deps; fi
	$(CARGO) fetch

up: ## Arranca el juego en modo desarrollo (compila rápido)
	$(CARGO) run

preview: ## Lanza la última build generada (sin recompilar)
	@latest=""; \
	for f in target/release/motorust target/release/motorust.exe target/debug/motorust target/debug/motorust.exe; do \
		if [ -f "$$f" ]; then \
			if [ -z "$$latest" ] || [ "$$f" -nt "$$latest" ]; then \
				latest="$$f"; \
			fi; \
		fi; \
	done; \
	if [ -n "$$latest" ]; then \
		echo "Ejecutando $$latest..."; \
		"$$latest"; \
	else \
		echo "No hay ninguna build generada. Ejecuta 'make up' o 'make build' primero."; \
		exit 1; \
	fi

build: ## Compila la versión optimizada en target/release/motorust
	$(CARGO) build --release

test: ## Pasa los tests
	$(CARGO) test

bench: ## Benchmark: recorre las pistas solo, con cada ajuste, y escribe los resultados (Esc lo para)
	$(CARGO) run --release -- --bench

linux-deps:
	@if command -v apt-get >/dev/null; then \
		$(SUDO) apt-get install -y $(APT_PACKAGES); \
	elif command -v dnf >/dev/null; then \
		$(SUDO) dnf install -y $(DNF_PACKAGES); \
	elif command -v pacman >/dev/null; then \
		$(SUDO) pacman -S --needed $(PACMAN_PACKAGES); \
	else \
		echo "Distribución no reconocida: instala a mano las dependencias de"; \
		echo "https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md"; \
	fi
