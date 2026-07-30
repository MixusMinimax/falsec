#!/usr/bin/env sh

cargo about generate --manifest-path=../Cargo.toml --config=./about.toml ./about.hbs > ../THIRD_PARTY_LICENSES
