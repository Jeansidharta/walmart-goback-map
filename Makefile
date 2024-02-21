build:
	cargo run
	mv positions.json ../front-end/src/assets/positions.json
	mv route.json ../front-end/src/assets/route.json
	cp plant.svg ../front-end/src/assets/plant.svg
