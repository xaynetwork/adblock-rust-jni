package com.xayn.adblockeraar

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test

class LoadingJniTest {

    @Test
    fun `Expect jni loads correctly and Hello World works`() {
        assertEquals("Hello Welt", Adblock.INSTANCE.hello("Welt"));
    }

    @Test
    fun `Expect to store string in rust`() {
        val pointer = Adblock.INSTANCE.store("Welt")
        println("Rust pointer $pointer")
        assertEquals("Welt", Adblock.INSTANCE.restore(pointer));
    }


    @Test
    fun `Expect to create a new Engine`() {
        assertNotNull(Adblock.INSTANCE.createEngine(""));
    }


}