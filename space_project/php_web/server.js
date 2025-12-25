const express = require('express');
const axios = require('axios');
const app = express();

// Настраиваем шаблонизатор EJS (чтобы вставлять данные в HTML)
app.set('view engine', 'ejs');
app.use(express.static('public'));

// ЭМУЛЯЦИЯ: прокси к Rust-сервису
app.get('/api/proxy/data', async (req, res) => {
    try {
        // Пытаемся получить данные от Rust сервиса
        // В докере будем обращаться по имени сервиса "rust_iss", а локально пока "localhost"
        const rustUrl = process.env.RUST_SERVICE_URL || 'http://localhost:3000';
        const response = await axios.get(`${rustUrl}/api/data`);
        res.json(response.data);
    } catch (error) {
        console.error("Ошибка подключения к Rust:", error.message);
        // Если Rust недоступен, отдаем заглушку, чтобы сайт не упал
        res.json([{
            id: 0, 
            source: 'frontend_stub', 
            data: { error: "Rust service unavailable" }, 
            fetched_at: new Date().toISOString()
        }]);
    }
});

// Главная страница дашборда
app.get('/', async (req, res) => {
    // Получаем данные (через нашу же прокси-функцию или напрямую)
    // Для простоты рендера передадим пустой массив, а данные подгрузим JS-ом на клиенте
    res.render('dashboard', { 
        title: 'Space Dashboard (Laravel Style)'
    });
});

const PORT = 8080;
app.listen(PORT, () => {
    console.log(`Frontend running on port ${PORT}`);
});
