import express from "express";
import path from "path";

const app = express();
const staticPath = path.resolve(__dirname, "client");

app.use("/", express.static(staticPath));

// Start the server
app.listen(8080, () => {
  console.log(`Server started on http://localhost:8080`);
});