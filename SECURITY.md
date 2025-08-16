 const express = require('express');
const bodyParser = require('body-parser');
const multer = require('multer');
const fs = require('fs-extra');
const { createCanvas, loadImage, registerFont } = require('canvas');
const { v4: uuidv4 } = require('uuid');
const path = require('path');

const app = express();
app.use(bodyParser.urlencoded({ extended: true }));
app.use(bodyParser.json());
app.use('/cards', express.static(path.join(__dirname, 'cards')));
app.use('/public', express.static(path.join(__dirname, 'public')));

// إعدادات رفع الصور
const upload = multer({ dest: 'uploads/' });

// ملفات البيانات
const DATA_FILE = path.join(__dirname, 'data', 'users.json');
fs.ensureFileSync(DATA_FILE);
let users = {};
try { users = fs.readJsonSync(DATA_FILE); } catch(e){ users = {}; }

// (اختياري) سجل خط مخصص للغة العربية إن لزم:
try {
  registerFont(path.join(__dirname, 'fonts', 'Amiri-Regular.ttf'), { family: 'Amiri' });
} catch(e) {
  // لو ما عندك الخط ما مشكلة
}

// صفحة التسجيل (نموذج بسيط)
app.get('/', (req, res) => {
  res.sendFile(path.join(__dirname, 'public', 'index.html'));
});

// استقبال التسجيل وإنشاء البطاقة
app.post('/register', upload.single('avatar'), async (req, res) => {
  try {
    const { name, rank, memberId } = req.body;
    if(!name || !rank || !memberId) return res.status(400).send('البيانات ناقصة');

    const id = uuidv4();
    const avatarPath = req.file ? req.file.path : null;

    // حفظ بيانات المستخدم
    users[id] = {
      id, name, rank, memberId, avatar: avatarPath ? req.file.filename : null,
      createdAt: new Date().toISOString()
    };
    await fs.writeJson(DATA_FILE, users, { spaces: 2 });

    // توليد صورة البطاقة
    const cardPath = await generateCardImage(users[id], avatarPath);
    res.json({ success: true, cardUrl: /cards/${path.basename(cardPath)}, id });
  } catch (err) {
    console.error(err);
    res.status(500).send('حدث خطأ داخلي');
  }
});

// دالة توليد بطاقة PNG باستخدام canvas
async function generateCardImage(user, avatarPath) {
  const width = 900, height = 540;
  const canvas = createCanvas(width, height);
  const ctx = canvas.getContext('2d');

  // خلفية
  ctx.fillStyle = '#0d1b2a';
  ctx.fillRect(0,0,width,height);

  // شريط علوي ذهبي
  ctx.fillStyle = '#f4d35e';
  ctx.fillRect(0,0,width,90);

  // شعار — لو عندك شعار كصورة ممكن تضعه هنا
  // ctx.drawImage(await loadImage('path/to/logo.png'), 30, 10, 70, 70);

  // نص العنوان
  ctx.fillStyle = '#fff';
  ctx.font = 'bold 36px Amiri, sans-serif';
  ctx.textAlign = 'center';
  ctx.fillText('بطاقة الهوية - Los Santos RP', width/2, 56);

  // مكان الصورة
  const avatarX = 60, avatarY = 120, avatarW = 220, avatarH = 220;
  ctx.fillStyle = '#1b263b';
  ctx.fillRect(avatarX-10, avatarY-10, avatarW+20, avatarH+20);

  if(avatarPath) {
    try {
      const img = await loadImage(avatarPath);
      // قص الدائرة للصورة
      ctx.save();
      ctx.beginPath();
      ctx.arc(avatarX + avatarW/2, avatarY + avatarH/2, avatarW/2, 0, Math.PI * 2);
      ctx.closePath();
      ctx.clip();
      ctx.drawImage(img, avatarX, avatarY, avatarW, avatarH);
      ctx.restore();
    } catch(e) {
      console.warn('avatar load failed', e);
    }
  } else {
    // أيقونة افتراضية
    ctx.fillStyle = '#ccc';
    ctx.fillRect(avatarX+20, avatarY+40, avatarW-40, avatarH-80);
  }

  // بيانات المستخدم
  ctx.fillStyle = '#f4d35e';
  ctx.font = 'bold 28px Amiri, sans-serif';
  ctx.textAlign = 'left';
  ctx.fillText('الاسم: ', 320, 160);
  ctx.fillStyle = '#fff';
  ctx.font = '24px Amiri, sans-serif';
  ctx.fillText(user.name, 420, 160);

  ctx.fillStyle = '#f4d35e';
  ctx.font = 'bold 28px Amiri, sans-serif';
  ctx.fillText('الرتبة: ', 320, 210);
  ctx.fillStyle = '#fff';
  ctx.font = '24px Amiri, sans-serif';
  ctx.fillText(user.rank, 420, 210);

  ctx.fillStyle = '#f4d35e';
