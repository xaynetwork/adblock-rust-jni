package com.xayn.adblockeraar;


import org.jetbrains.annotations.NotNull;

import java.io.IOException;
import java.io.InputStream;

import androidx.annotation.NonNull;

public enum Adblock implements AdblockService {
    INSTANCE;

    private class AdblockEngineImpl implements AdblockEngine {

        private final long pointer;
        private boolean destroyed;

        private AdblockEngineImpl(long pointer) {
            this.pointer = pointer;
        }

        @Override
        public AdblockResult match(@NonNull String url, @NonNull String hostname, @NonNull String type) {
            guard();
            byte res = Adblock.this.simpleMatch(pointer, url, hostname, type);
            return new AdblockResult(res);
        }

        @Override
        public boolean deserialize(InputStream stream) throws IOException {
            byte[] targetArray = new byte[stream.available()];
            if (stream.read(targetArray) == -1) {
                return false;
            }
            return engineDeserialize(pointer, targetArray);
        }

        @Override
        public boolean deserialize(String filePath) {
            return engineDeserializeFromFile(pointer, filePath);
        }

        @Override
        public void enableTag(@NotNull String tag) {
            engineEnableTag(pointer, tag);
        }

        @Override
        public void disableTag(@NotNull String tag) {
            engineDisableTag(pointer, tag);
        }

        @Override
        public boolean hasTag(@NotNull String tag) {
            return engineTagExists(pointer, tag);
        }

        @Override
        public void destroy() {
            engineDestroy(pointer);
            destroyed = true;
        }

        @Override
        protected void finalize() throws Throwable {
            destroy();
            super.finalize();
        }

        private void guard() {
            if (destroyed) {
                throw new RuntimeException("Engine was used after calling destroyed");
            }
        }
    }

    static {
        System.loadLibrary("adblockerjni");
    }

    private native byte match(long engine, String url, String hostname, String sourceHostName, boolean thirdParty, String resourceType, byte previousResult);

    private native byte simpleMatch(long engine, String url, String sourceHostName, String resourceType);

    private native long engineCreate(String rules);

    private native long engineCreateDefault();

    private native void engineDestroy(long pointer);

    private native void engineEnableTag(long pointer, String tag);

    private native boolean engineTagExists(long pointer, String tag);

    private native void engineDisableTag(long pointer, String tag);

    private native boolean engineAddResource(long pointer, String key, String contentType, String data);

    private native void engineAddResourcesFromJson(long pointer, String resourcesJson);

    private native boolean engineDeserialize(long pointer, byte[] data);

    private native boolean engineDeserializeFromFile(long pointer, String filePath);


    @Override
    public AdblockEngine createEngine(String rules) {
        if (rules == null || rules.isEmpty()) {
            return createEngine();
        }
        long pointer = engineCreate(rules);
        return new AdblockEngineImpl(pointer);
    }

    @Override
    public AdblockEngine createEngine() {
        long pointer = engineCreateDefault();
        return new AdblockEngineImpl(pointer);
    }
}



