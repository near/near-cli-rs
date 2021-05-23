near-cli
--------
near-cli – это интерактивный помощник составления и выполнения команд для транзакций. В целом новичку трудно сразу разобраться как устроены команды.
For example, I consider having the following command to do a transfer:
```
near-cli transfer near \
    network testnet \
    sender 'volodymyr.testnet' \
    receiver '21.volodymyr.testnet' \
    amount  '1 NEAR' \
    sign-with-keychain \
    send
```
Это полная команда. Результат ее работы будет такой:
```
---  Success:  ---
 FinalExecutionOutcome {
    status: SuccessValue(``),
    ...
}
```
Набирая эту или другую команду вручную, Вы можете допустить ошибку, либо забыть последовательность набора команды.
Не проблема. – help подскажет как правильно выстроить команду.
Однако, используя near-cli, Вы в любом месте набора команды можете нажать Enter и интерактивный режим программы продолжит работу по составлению команды с того места, где Вы закончили вводить необходимые параметры.
