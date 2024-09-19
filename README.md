<h1>Вебсокеты на ActixWeb</h1>

<p>
    Быстрый запуск: 
    <code>cargo run</code>
</p>

<p>
    Проверка функционала производилась посредством <b>Postman</b>
</p>

<h2>Команды</h2>

<ul>
    <li>
        Добавление: 
        <code>
            {"action": "add_book", "book": {"title": "Test Book", "author": "I am", "year": 2024}}
        </code>
    </li>
    <li>
        Получение всех: 
        <code>
            {"action": "get_books"}
        </code>
    </li>
    <li>
        Получение по идентификатору: 
        <code>
            {"action": "get_book", "id": "123456"}
        </code>
    </li>
    <li>
        Обновление: 
        <code>
            {"action": "update_book", "id": "123456", "book": {"title": "Test Book 2", "author": "You are", "year": 2024}}
        </code>
    </li>
    <li>
        Удаление: 
        <code>
            {"action": "delete_book", "id": "123456"}
        </code>
    </li>
</ul>
