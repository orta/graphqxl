const { graphqxlToSdl } = require("./index.js");

// Example 1: Basic schema
console.log("=== Example 1: Basic Schema ===");
const basicVfs = {
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

const basicResult = graphqxlToSdl(basicVfs, "schema.graphqxl");
console.log(basicResult);
console.log();

// Example 2: Schema with included imports
console.log("=== Example 2: Schema with Imports ===");
const importVfs = {
  "schema.graphqxl": `
    import "models/user"
    import "models/post"
    
    type Query {
      user(id: ID!): User
      post(id: ID!): Post
    }
  `,
  "models/user.graphqxl": `
    type User {
      id: ID!
      name: String!
      posts: [Post!]!
    }
  `,
  "models/post.graphqxl": `
    type Post {
      id: ID!
      title: String!
      author: User!
    }
  `,
};

const importResult = graphqxlToSdl(importVfs, "schema.graphqxl");
console.log(importResult);
console.log();

// Example 3: Using generics
console.log("=== Example 3: Using Generics ===");
const genericVfs = {
  "schema.graphqxl": `
    type Response<T> {
      data: T
      error: String
    }
    
    type User {
      id: ID!
      name: String!
    }
    
    type UserResponse = Response<User>
    
    type Query {
      getUser(id: ID!): UserResponse!
    }
  `,
};

const genericResult = graphqxlToSdl(genericVfs, "schema.graphqxl");
console.log(genericResult);
console.log();

// Example 4: With custom indent and private prefix
console.log("=== Example 4: Custom Options ===");
const optionsVfs = {
  "schema.graphqxl": `
    type User {
      id: ID!
      name: String!
      __privateField: String
    }
    
    type Query {
      user(id: ID!): User
    }
  `,
};

const optionsResult = graphqxlToSdl(optionsVfs, "schema.graphqxl", 4, "__");
console.log(optionsResult);
