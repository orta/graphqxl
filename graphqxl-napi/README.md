# @orta/graphqxl

Node.js bindings for [GraphQXL](https://github.com/gabotechs/graphqxl), a GraphQL schema language extension that provides additional features like imports, generics, and inheritance.

## Installation

```bash
npm install @orta/graphqxl
# or
yarn add @orta/graphqxl
```

## Usage

This package provides a single function `graphqxlToSdl` that takes a virtual file system (VFS) object and converts GraphQXL schemas to standard GraphQL SDL.

### Basic Example

```javascript
import { graphqxlToSdl } from "@orta/graphqxl";

// Create a virtual file system with your GraphQXL schemas
const vfs = {
  "schema.graphqxl": `
    type User {
      id: ID!
      name: String!
      email: String!
    }
    
    type Query {
      user(id: ID!): User
      users: [User!]!
    }
  `,
};

// Convert to GraphQL SDL
const sdl = graphqxlToSdl(vfs, "schema.graphqxl");
console.log(sdl);
```

### With Imports

```javascript
const vfs = {
  "schema.graphqxl": `
    import "user"
    import "post"
    
    type Query {
      user(id: ID!): User
      users: [User!]!
      post(id: ID!): Post
      posts: [Post!]!
    }
  `,
  "user.graphqxl": `
    type User {
      id: ID!
      name: String!
      email: String!
      posts: [Post!]!
    }
  `,
  "post.graphqxl": `
    type Post {
      id: ID!
      title: String!
      content: String!
      author: User!
    }
  `,
};

const sdl = graphqxlToSdl(vfs, "schema.graphqxl");
```

### With Generics

```javascript
const vfs = {
  "schema.graphqxl": `
    type Connection<T> {
      edges: [Edge!]!
      pageInfo: PageInfo!
    }
    
    type Edge<T> {
      node: T!
      cursor: String!
    }
    
    type PageInfo {
      hasNextPage: Boolean!
      endCursor: String
    }
    
    type User {
      id: ID!
      name: String!
    }
    
    type UserConnection = Connection<User>
    type UserEdge = Edge<User>
    
    type Query {
      users: UserConnection!
    }
  `,
};

const sdl = graphqxlToSdl(vfs, "schema.graphqxl");
```

## API

### `graphqxlToSdl(vfs, entryPath, indentSpaces?, privatePrefix?)`

Converts GraphQXL schema files to standard GraphQL SDL.

#### Parameters:

- `vfs` (object): A virtual file system object where keys are file paths and values are file contents
- `entryPath` (string): The entry point file path in the VFS
- `indentSpaces` (number, optional): Number of spaces for indentation (default: 2)
- `privatePrefix` (string, optional): Prefix for private fields (default: "\_")

#### Returns:

- (string): The generated GraphQL SDL

## Features Supported

- ✅ Basic GraphQL types (type, input, enum, interface, union, scalar)
- ✅ Import system for modular schemas
- ✅ Generic types with type parameters
- ✅ Type inheritance and composition
- ✅ Schema extensions
- ✅ Private fields (with configurable prefix)
- ✅ Directives

## License

MIT
