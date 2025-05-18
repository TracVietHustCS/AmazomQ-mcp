# PostgreSQL MCP Server for Amazon Q 

A Rust-based Modular Capability Provider (MCP) server that enables Amazon Q interact PostgreSQL database. Server allows users to query database struture read-only queries throught the Amazon Q chat Interface. 


## Overview 
This project implements a PostgreSQL server that follows the MCP protocol used by Amazon Q CLI
1. List all table in a PostgreSQL databases 
2. Get Schema info for specific tables
3. Execute read-only SQL queries with result return as JSON 

## Prerequisites 
- Rust 1.70 
- Cargo package manager
``` bash
cargo new postgresql_server
```
- PostgreSQL database 
- Amazon Q CLI 

## Installation 

1. Clone the Repository: 
```bash
git clone ...
cd postgresql-mcp-server
```

2. Build the project : 
```bash 
cargo build --release --bin postgresql_server
```
<img width="765" alt="BUILD SUCCESSFUL" src="https://github.com/user-attachments/assets/305d1fd2-a4c9-4b2c-89a7-34c6d1d8f4ac" />


## Usage 
### Running the Server 
- Server requires a PostgreSQL connection string as a command-line-argument : 
```bash 
./target/release/postgresql_server "postgresql://username:password@hostname:port/database"
```

### Intergrating with Amazo Q CLI 
1. Create or edit the MCP 

```bash
vim ~/.aws/amazonq/mcp.json
```

2. Add the PosgreSQL server configuration to file 

```json
{
  "mcpServers": {
    "calculator": {
      "command": "/home/ec2-user/sample-building-mcp-servers-with-rust/target/release/calculator_server",
      "args": []
    },
    "rds": {
      "command": "/home/ec2-user/sample-building-mcp-servers-with-rust/target/release/rds_server",
      "args": []
    },
    "s3": {
      "command": "/home/ec2-user/sample-building-mcp-servers-with-rust/target/release/s3_server",
      "args": []
    },
    "postgres": {
      "command": "/home/ec2-user/sample-building-mcp-servers-with-rust/target/release/postgresql_server",
      "args": [
        "postgresql://postgres:<DB-PASSWORD>@<DB-ENDPOINT>.com:5432/demo"
      ]
    }
  }
}
```

3. Testing Q Amanzon Server
-  Start Q Amazon 
```bash
qchat
```
<img width="673" alt="image" src="https://github.com/user-attachments/assets/f41a16e4-942b-41cb-867a-77c68c0b58dd" />

- Ask Amazon Q to list tables in the database 
`Can you list all tables in my PostgreSQL database using the postgres operator?`
<img width="691" alt="image" src="https://github.com/user-attachments/assets/f93abdb6-72a7-4f16-a317-ba68eed488b9" />

- Try asking for schema of specific table: 
`What columns are in the student table? `
<img width="814" alt="image" src="https://github.com/user-attachments/assets/dac5a65b-e1a1-4a82-96de-3caf9a0abd86" />


- Execute a SQL query: 
`List 100 students sorted by name.`
<img width="716" alt="image" src="https://github.com/user-attachments/assets/0c49c9c8-580d-426b-bdd9-ed12b21d24eb" />

## Project Structure 
- `src/main.rs`: Entry point that comman-line-arguments, logging setup, and server initialization 
- `src/operator.rs` Implementation of the PostgreSQL with database interaction tools.

## Tool Implementation in `operator.rs` 
The server implements three main tools using the tool artribute from the RMCP framework: 

1. List Tables: 
```rust 
#[tool(description = "List tables in the PostgreSQL database")]
async fn list_tables(&self) -> String {
    // Implementation that queries information_schema.tables
    // and returns a JSON string of table names
}
```

2. Get Table Schema : 
```rust 
#[tool(description = "Get schema for a specific table")]
async fn get_table_schema(
    &self,
    #[tool(param)]
    #[schemars(description = "Name of the table to get schema for")]
    table_name: String,
) -> String {
    // Implementation that queries information_schema.columns
    // and returns a JSON string of column names and data types
}

```

3. Execute SQL Query: 
```rust 
#[tool(description = "Run a read-only SQL query")]
async fn query(
    &self,
    #[tool(param)]
    #[schemars(description = "SQL query to execute (read-only)")]
    sql: String,
) -> String {
    // Implementation that executes the SQL query
    // and returns a JSON string of the results
}
```









