import test from "ava";

import { graphqxlToSdl } from "../index.js";

test("graphqxlToSdl basic test", (t) => {
  // Create a virtual file system object
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
    `
  };
  
  const result = graphqxlToSdl(vfs, "schema.graphqxl", 2, "_");
  
  // The result should be valid GraphQL SDL
  t.truthy(result);
  t.true(result.includes("type User"));
  t.true(result.includes("type Query"));
  t.true(result.includes("id: ID!"));
});

test("graphqxlToSdl with imports", (t) => {
  // Create a VFS with multiple files
  const vfs = {
    "schema.graphqxl": `
      import "user"
      
      type Query {
        user(id: ID!): User
        users: [User!]!
      }
    `,
    "user.graphqxl": `
      type User {
        id: ID!
        name: String!
        email: String!
      }
    `
  };
  
  const result = graphqxlToSdl(vfs, "schema.graphqxl", 2, "_");
  
  t.truthy(result);
  t.true(result.includes("type User"));
  t.true(result.includes("type Query"));
});

test("graphqxlToSdl with generics", (t) => {
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
    `
  };
  
  const result = graphqxlToSdl(vfs, "schema.graphqxl", 2, "_");
  
  t.truthy(result);
  // The output should have expanded the generic types
  t.true(result.includes("type Query"));
  t.true(result.includes("type UserConnection"));
  t.true(result.includes("type UserEdge"));
});
