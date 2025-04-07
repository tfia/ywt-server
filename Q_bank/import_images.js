const fs = require('fs');

const conn = new Mongo("localhost:27017");
const db = conn.getDB("ywt_db");

const qBankContent = fs.readFileSync('./Q_bank.json', 'utf8');
const Q_bank = JSON.parse(qBankContent);

print("Starting image import to collection 'qbank'...");

let importCount = 0;
let errorCount = 0;

Q_bank.forEach(item => {
  try {
    const { id, tags, path: imagePath } = item;
    
    const imageBuffer = fs.readFileSync(imagePath);
    
    const doc = {
      _id: id,
      tags: tags,
      image: Binary(imageBuffer)  // Use the MongoDB shell's Binary function
    };
    
    db.qbank.updateOne(
      { _id: id },
      { $set: doc },
      { upsert: true }
    );
    
    print(`Imported image ${id}: ${imagePath}`);
    importCount++;
  } catch (err) {
    print(`Failed to import image ${JSON.stringify(item)}: ${err.message}`);
    errorCount++;
  }
});

print(`Import completed. Imported ${importCount} images. Errors: ${errorCount}.`);