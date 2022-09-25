Rust JWT User Authentication Server built with actix

### API
- Health endpoint: `GET` /
  ```
  curl http://localhost:3000/callback
  ```
- User Signup: `POST` /signup
  ```
  curl --request POST \
    --url http://localhost:3000/register \
    --header 'content-type: application/json' \
    --data '{
        "username": "user1",
        "email": "user1@example.com",
        "password": "user1"
    }'
  ```
- Auth: `POST` /auth
  ```
  curl --request POST \
    --url http://localhost:3000/auth \
    --user user1
  ```
- User profile: `GET` /me
  ```
  curl --request GET \
  --url http://localhost:3000/me \
  --header 'authorization: Bearer <jwt_token>'
  ```
