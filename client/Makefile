PROTOC_GEN_TS_PATH = "node_modules/.bin/protoc-gen-ts"
OUT_DIR = "./src/app/protobuf/"

default: all

schema:
	mkdir -p $(OUT_DIR)
	protoc \
		--plugin="protoc-gen-ts=$(PROTOC_GEN_TS_PATH)" \
		--js_out="import_style=commonjs,binary:$(OUT_DIR)" \
		--ts_out="$(OUT_DIR)" \
		--proto_path="../schema/" \
		../schema/*.proto

all:
	make schema
