.PHONY: clean build package deploy

export TARGET_CC := clang

build:
	cargo build --target aarch64-unknown-linux-musl --release

package: clean build
	mkdir -pv ./package
	mkdir -pv ./package/tmp

	# fetch
	mv target/aarch64-unknown-linux-musl/release/fetch ./package/tmp/bootstrap
	zip package/fetch.zip --filesync --junk-paths ./package/tmp/bootstrap
	# alert
	mv target/aarch64-unknown-linux-musl/release/alert ./package/tmp/bootstrap
	zip package/alert.zip --filesync --junk-paths ./package/tmp/bootstrap

clean:
	rm -rf ./package/*zip
	rm -f ./package/tmp/bootstrap

deploy: clean build package
	serverless deploy --verbose
