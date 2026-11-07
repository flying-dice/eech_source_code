# Task-local derivative for EEDCS-49 tree raw-word pass-through: the immutable base
# (rust 1.99.1, gcc-12, x86_64 + windows-gnu targets; no 32-bit gcc runtime) plus exactly what the CI i686 job's Toolchain step adds
# (gcc-multilib gcc-12-multilib, dpkg --add-architecture i386, liblua5.1-0-dev:i386, rustup target add i686-unknown-linux-gnu).
FROM eech-linux-sdk-fp-base:4a5618cbc1
RUN dpkg --add-architecture i386 \
	&& apt-get update \
	&& apt-get install -y --no-install-recommends gcc gcc-multilib gcc-12 gcc-12-multilib liblua5.1-0-dev:i386 \
	&& rm -rf /var/lib/apt/lists/* \
	&& rustup target add i686-unknown-linux-gnu
