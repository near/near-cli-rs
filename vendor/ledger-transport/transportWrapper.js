
export class TransportWrapper {
    exchange(apduCommand) {
        return original_transport.exchange(apduCommand);
    }
}
