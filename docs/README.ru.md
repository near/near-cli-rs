near-cli
--------
near-cli – это утилита командной строки для работы с блокчейном Near Protocol.

### README.md

* en [English](README.en.md)

### Оглавление

* [Применение](#применение)
* [Инструкция](GUIDE.ru.md)
* [Установка](#установка)
* [Сборка](#сборка)

### Применение

Вцелом новичку трудно сразу разобраться как устроены команды.  
Например, для осуществления перевода токенов необходимо набрать в терминале такую команду:
```txt
./near-cli transfer near \
    network testnet \
    sender 'volodymyr.testnet' \
    receiver '21.volodymyr.testnet' \
    amount  '1 NEAR' \
    sign-with-keychain \
    send
```
Это полная команда. Результат ее работы будет такой:
```txt
---  Success:  ---
 FinalExecutionOutcome {
    status: SuccessValue(``),
    ...
}
```
Набирая эту или другую команду вручную, Вы можете допустить ошибку, либо забыть последовательность набора команды.  
Не проблема. – help подскажет как правильно выстроить команду.  
Однако, используя near-cli, Вы __в любом месте набора команды__ можете нажать Enter и интерактивный режим программы продолжит работу по составлению команды с того места, где Вы закончили вводить необходимые параметры.

<details><summary><i>Демонстрация работы утииты с частично набранной командой</i></summary>
<a href="https://asciinema.org/a/tdNu6qoDKUzFH6ZCsfADHoqOP?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/tdNu6qoDKUzFH6ZCsfADHoqOP.png" width="836"/>
</a>
</details>



### Установка

### Сборка
