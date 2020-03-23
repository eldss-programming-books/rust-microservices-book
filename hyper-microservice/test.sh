# Very simple tests for the server

# Add three users
curl -X POST http://localhost:8080/user/
printf "\n"
curl -X POST http://localhost:8080/user/
printf "\n"
curl -X POST http://localhost:8080/user/
printf "\n"
# Expect output (on first run):
#   0
#   1
#   2

# Remove a user
curl -X DELETE http://localhost:8080/user/1
# List remaining users
curl http://localhost:8080/users
printf "\n"
# Expect output:
#   0,2