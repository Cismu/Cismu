<script setup>
import { ref } from "vue";

const isPlaying = ref(false);

const play = () => {
    isPlaying.value = true;
};

const pause = () => {
    isPlaying.value = false;
};
</script>

<template>
    <div>
        <h1>Componente Main</h1>
        <button v-if="!isPlaying" @click="play">Play</button>
        <button v-else @click="pause">Pause</button>
    </div>
</template>
