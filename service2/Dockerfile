FROM ghcr.io/astral-sh/uv:python3.10-bookworm-slim

WORKDIR /app

# Enable bytecode compilation and set linking mode
ENV UV_COMPILE_BYTECODE=1
ENV UV_LINK_MODE=copy

# Install project dependencies using the lockfile and settings
RUN --mount=type=cache,target=/root/.cache/uv \
    --mount=type=bind,source=uv.lock,target=uv.lock \
    --mount=type=bind,source=pyproject.toml,target=pyproject.toml \
    uv sync --frozen --no-install-project --no-dev

# Add the rest of the project source code and install it
ADD . /app
RUN --mount=type=cache,target=/root/.cache/uv \
    uv sync --frozen --no-dev

# Prepend the virtual environment's bin directory to PATH
ENV PATH="/app/.venv/bin:$PATH"

ENTRYPOINT []

CMD ["recieve"]