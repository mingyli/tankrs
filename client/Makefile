schema:
	flatc  --ts --no-fb-import -o src/flatschema ../schema/*.fbs
	sed -i.old "1s;^;import {flatbuffers} from 'flatbuffers';" src/flatschema/*

dev:
	yarn run typecheck
	yarn run build

all:
	make schema
	make dev