use anyhow::{Context, Result};
use rmcp::{
    ServerHandler,
    model::{ServerCapabilities, ServerInfo},
    tool,
};
use serde::Serialize;
use tokio_postgres::{Client, NoTls};
use schemars::JsonSchema;

// Let's use String as our return type since it's already supported by rmcp
// We'll serialize our data to JSON strings

#[derive(Debug, Serialize, JsonSchema)]
pub struct TableInfo {
    table_name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ColumnInfo {
    column_name: String,
    data_type: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct QueryResult {
    rows: Vec<serde_json::Value>,
}


//PostgresOperator
// The core struct that manages the connection to the PostgreSQL database and implements the tool 
#[derive(Debug, Clone)]
pub struct PostgresOperator {
    connection_string: String,
}

impl PostgresOperator {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }

    //Database connection 
    //get_client method establishes a connect to the PostgrSQL databes 
    // Helper method to create a PostgreSQL client
    async fn get_client(&self) -> Result<Client> {
        let (client, connection) = tokio_postgres::connect(&self.connection_string, NoTls)
            .await
            .context("Failed to connect to PostgreSQL")?;

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("Connection error: {}", e);
            }
        });

        Ok(client)
    }
}



#[tool(tool_box)]
impl PostgresOperator {
    #[tool(description = "List tables in the PostgreSQL database")]
    async fn list_tables(&self) -> String {
        tracing::info!("Listing tables in PostgreSQL database...");

        match self.get_client().await {
            Ok(client) => {
                match client
                    .query(
                        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'",
                        &[],
                    )
                    .await
                {
                    Ok(rows) => {
                        let tables = rows
                            .iter()
                            .map(|row| {
                                let table_name: String = row.get("table_name");
                                TableInfo { table_name }
                            })
                            .collect::<Vec<TableInfo>>();

                        tracing::info!("Successfully listed {} tables", tables.len());
                        serde_json::to_string(&tables).unwrap_or_else(|_| "[]".to_string())
                    }
                    Err(err) => {
                        tracing::error!("Failed to list tables: {:?}", err);
                        "[]".to_string()
                    }
                }
            }
            Err(err) => {
                tracing::error!("Failed to connect to database: {:?}", err);
                "[]".to_string()
            }
        }
    }

    #[tool(description = "Get schema for a specific table")]
    async fn get_table_schema(
        &self,
        #[tool(param)]
        #[schemars(description = "Name of the table to get schema for")]
        table_name: String,
    ) -> String {
        tracing::info!("Getting schema for table: {}", table_name);
        
        match self.get_client().await {
            Ok(client) => {
                let query = r#"
                    SELECT column_name, data_type
                    FROM information_schema.columns
                    WHERE table_name = $1
                "#;

                match client.query(query, &[&table_name]).await {
                    Ok(rows) => {
                        let columns = rows
                            .iter()
                            .map(|row| ColumnInfo {
                                column_name: row.get("column_name"),
                                data_type: row.get("data_type"),
                            })
                            .collect::<Vec<_>>();
                        
                        tracing::info!("Successfully retrieved schema for table {} with {} columns", table_name, columns.len());
                        serde_json::to_string(&columns).unwrap_or_else(|_| "[]".to_string())
                    }
                    Err(err) => {
                        tracing::error!("Failed to get schema: {:?}", err);
                        "[]".to_string()
                    }
                }
            }
            Err(err) => {
                tracing::error!("Failed to connect to database: {:?}", err);
                "[]".to_string()
            }
        }
    }

    #[tool(description = "Run a read-only SQL query")]
    async fn query(
        &self,
        #[tool(param)]
        #[schemars(description = "SQL query to execute (read-only)")]
        sql: String,
    ) -> String {
        tracing::info!("Executing SQL query: {}", sql);
        
        // Check if the query is read-only (starts with SELECT)
        if !sql.trim().to_uppercase().starts_with("SELECT") {
            tracing::error!("Only SELECT queries are allowed for security reasons");
            return serde_json::to_string(&QueryResult { rows: vec![] }).unwrap_or_else(|_| "{\"rows\":[]}".to_string());
        }
        
        match self.get_client().await {
            Ok(client) => {
                match client.query(&sql, &[]).await {
                    Ok(rows) => {
                        let mut result = Vec::new();
                        for row in rows {
                            let mut map = serde_json::Map::new();
                            for (i, column) in row.columns().iter().enumerate() {
                                let name = column.name();
                                // Remove unused variable warning
                                // let column_type = column.type_();
                                let value = if let Ok(text) = row.try_get::<_, String>(i) {
                                    // Convert the string to JSON if possible, otherwise use as string
                                    serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text))
                                } else if let Ok(num) = row.try_get::<_, i32>(i) {
                                    serde_json::Value::Number(num.into())
                                } else if let Ok(num) = row.try_get::<_, i64>(i) {
                                    // Convert i64 to serde_json::Number via string to avoid potential overflow
                                    serde_json::Value::String(num.to_string())
                                } else if let Ok(b) = row.try_get::<_, bool>(i) {
                                    serde_json::Value::Bool(b)
                                } else {
                                    serde_json::Value::Null
                                };
                                map.insert(name.to_string(), value);
                            }
                            result.push(serde_json::Value::Object(map));
                        }
                        
                        tracing::info!("Query executed successfully, returned {} rows", result.len());
                        serde_json::to_string(&QueryResult { rows: result }).unwrap_or_else(|_| "{\"rows\":[]}".to_string())
                    }
                    Err(err) => {
                        tracing::error!("Query execution failed: {:?}", err);
                        serde_json::to_string(&QueryResult { rows: vec![] }).unwrap_or_else(|_| "{\"rows\":[]}".to_string())
                    }
                }
            }
            Err(err) => {
                tracing::error!("Failed to connect to database: {:?}", err);
                serde_json::to_string(&QueryResult { rows: vec![] }).unwrap_or_else(|_| "{\"rows\":[]}".to_string())
            }
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for PostgresOperator {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A PostgreSQL database server".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
