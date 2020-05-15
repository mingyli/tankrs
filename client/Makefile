schema:
	flatc  --ts --no-fb-import -o src/flatschema ../schema/actions.fbs ../schema/math.fbs ../schema/messages.fbs ../schema/world.fbs

dev:
	yarn run typecheck
	yarn run build