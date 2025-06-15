import { graphqxlToSdl } from './index';

// Define a type for the VFS to get better type safety
type VirtualFileSystem = Record<string, string>;

// Example with type safety
const vfs: VirtualFileSystem = {
  'schema.graphqxl': `
    # GraphQXL supports advanced features
    
    interface Node {
      id: ID!
    }
    
    type User implements Node {
      id: ID!
      name: String!
      email: String!
    }
    
    # Generic type for pagination
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
      hasPreviousPage: Boolean!
      startCursor: String
      endCursor: String
    }
    
    # Concrete type using generics
    type UserConnection = Connection<User>
    type UserEdge = Edge<User>
    
    type Query {
      users(first: Int, after: String): UserConnection!
      user(id: ID!): User
    }
  `
};

try {
  const sdl = graphqxlToSdl(vfs, 'schema.graphqxl', 2, '_');
  console.log('Generated GraphQL SDL:');
  console.log(sdl);
  
  // You can now use this SDL with any GraphQL server
  // For example with Apollo Server:
  // const server = new ApolloServer({ typeDefs: sdl });
} catch (error) {
  console.error('Error converting GraphQXL to SDL:', error);
}