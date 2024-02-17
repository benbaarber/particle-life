import express from "express";
import path from "path";

const app = express();

app.use("/", express.static(__dirname))

// Serve the HTML file
app.get('/', (req, res) => {
  res.sendFile(path.join(__dirname, 'life.html'));
});

// Start the server
app.listen(8080, () => {
  console.log(`Server started on http://localhost:8080`);
});