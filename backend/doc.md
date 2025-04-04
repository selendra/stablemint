1. Register:

```sh
mutation {
  register(input: {
    name: "LayNath242",
    username: "laynath242",
    email: "laynath242@example.com",
    password: "securepassword123"
  }) {
    token
    user {
      id
      name
      username
      email
      address
      createdAt
    }
  }
}
```

2. Login:

```sh
mutation {
  login(input: {
    username: "laynath242",
    password: "securepassword123"
  }) {
    token
    user {
      id
      name
      username
      email
      address
      createdAt
    }
  }
}
```

3. Get Current User:

```sh
query {
  me {
    id
    name
    username
    email
    address
    createdAt
  }
}
```
{
  "Authorization": "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiLin6hjYjAwMTRjYi1iMjJmLTRlMjMtYTdjYS0xN2VmODNlZmFkZWbin6kiLCJleHAiOjE3NDM4NDEzMTksImlhdCI6MTc0Mzc1NDkxOSwidXNlcm5hbWUiOiJsYXluYXRoMjRzMiJ9.3SEI-HUXIUjjlcQfY_uS9LkGEkTBuSoFgtRurfuZWso"
}