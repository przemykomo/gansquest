package przemyk.gansquest;

import org.libsdl.app.SDLActivity;
import android.content.Intent;
import android.net.Uri;

public class MyActivity extends SDLActivity {
    public static boolean pickDir = false;
    protected String[] getLibraries() {
        return new String[] { "SDL3", "gansquest" };
    }

    // Spaghetti based on https://github.com/libsdl-org/SDL/pull/9687
    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (((MyActivity)mSingleton).pickDir == true) {
            ((MyActivity)mSingleton).pickDir = false;
            System.out.println("ABOBA ABOBA ABOBA ABOBA");
            System.out.println(data.getData().getPath()); //TODO: null check?
        }
    }

    // waiting for https://github.com/libsdl-org/SDL/issues/9657
    public static void pickDirectory() {
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT_TREE);
        mSingleton.startActivityForResult(intent, 0); // request code guess
        ((MyActivity)mSingleton).pickDir = true;
        //startActivityForResult(intent, REQUEST_CODE_PICK_CUSTOM_DIRECTORY);
    }
}
