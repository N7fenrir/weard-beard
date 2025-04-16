FROM python:3.10-slim AS builder
WORKDIR /app
RUN pip install --upgrade pip build
COPY . .
RUN python -m build --wheel --outdir=dist .

FROM python:3.10-slim
WORKDIR /app
COPY --from=builder /app/dist/service2-0.1.0-py3-none-any.whl .
RUN pip install --no-cache-dir service2-0.1.0-py3-none-any.whl
ENTRYPOINT ["recieve"]
