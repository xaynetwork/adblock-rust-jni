package com.xayn.adblockeraar;

public enum Adblock implements AdblockService {
    INSTANCE;

    private class AdblockEngineImpl implements  AdblockEngine{
        final long pointer;
        private AdblockEngineImpl(long pointer) {
            this.pointer = pointer;
        }
    }

    static {
        System.loadLibrary("adblockerjni");
    }

    @Override
    public native String hello(String message);

    @Override
    public native long store(String message);

    @Override
    public native String restore(long message);

    @Override
    public native void init(String filePath);

    @Override
    public native synchronized boolean match(String url) ;

    private native long engineCreate(String rules);

    @Override
    public AdblockEngine createEngine(String rules) {
        long pointer = engineCreate(rules);
        return new AdblockEngineImpl(pointer);
    }


}

interface AdblockService {
    String hello(String message);
    long store(String message);
    String restore(long message);
    void init(String filePath);
    boolean match(String url);

    AdblockEngine createEngine(String rules);
}



interface AdblockEngine {}