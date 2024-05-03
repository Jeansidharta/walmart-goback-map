build:
	cargo run
	mv positions.json ../walmart-goback-frontend/src/assets/positions.json
	mv route.json ../walmart-goback-frontend/src/assets/route.json
	cp plant.svg ../walmart-goback-frontend/src/assets/plant.svg
