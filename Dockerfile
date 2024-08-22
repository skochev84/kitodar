FROM rust:latest as build

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk wasm-bindgen-cli

WORKDIR /usr/src/kitodar
COPY . .

RUN cd frontend && trunk build --release
RUN cd backend && cargo build --release

FROM rust:slim

COPY --from=build /usr/src/kitodar/backend/target/release/backend /usr/local/bin/backend
COPY --from=build /usr/src/kitodar/frontend/dist /usr/local/bin/dist

WORKDIR /usr/local/bin
CMD ["backend"]