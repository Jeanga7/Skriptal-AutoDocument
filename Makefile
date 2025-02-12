run-back:
	docker ps -a --filter "name=my-postgres" --format "{{.Names}}" | grep -q "my-postgres" || docker run --name my-postgres -e POSTGRES_PASSWORD=your_super_secret_key -d postgres
	docker start my-postgres
	cd backend && cargo r --release
