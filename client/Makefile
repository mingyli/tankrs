schema:
	flatc  --ts --no-fb-import -o src/flatschema ../schema/*.fbs

dev:
	yarn run typecheck
	yarn run build

all:
	make schema
	make dev